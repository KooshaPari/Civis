# API Reference

<CategorySwitcher />

This page documents the public API that exists in the current Civis workspace.
It intentionally separates shipped Rust APIs from planned protocol surfaces so
agents do not treat future WebSocket/JSON-RPC specs as implemented code.

## Current Status

| Surface | Status | Source |
| --- | --- | --- |
| Rust engine crate | Implemented | `crates/engine/src/lib.rs` |
| Rust CLI/server binary | Implemented as a smoke executable | `crates/server/src/main.rs` |
| WebSocket JSON-RPC protocol | Implemented (`civ-server`) | [`jsonrpc-surface.md`](jsonrpc-surface.md) · `docs/specs/CIV-0200-client-protocol.md` |
| Metrics export endpoint | Planned | `docs/traceability/EVENT_TAXONOMY.md` |

## Engine Crate

Import the engine crate as `civ_engine`.

```rust
use civ_engine::{Fixed, WorldState, metrics, step};

let state = WorldState::default();
let next = step(state, Fixed::from_num(1_000));
let summary = metrics::compute(
    next.energy_budget_joules.to_f64(),
    Fixed::from_num(1_000).to_f64(),
);
```

### Fixed

`Fixed` is the deterministic numeric type used by the simulation. Values are
stored as scaled `i64` integers with `SCALE = 1_000_000`.

| API | Description |
| --- | --- |
| `Fixed::ZERO` | Constant zero value. |
| `Fixed::ONE` | Constant one value. |
| `Fixed::from_num(value)` | Converts an integer-like value into scaled fixed point. |
| `Fixed::from_raw(raw)` | Wraps an already scaled raw value. |
| `to_f64()` | Converts to floating point for display or diagnostics. |
| `saturating_add(other)` | Adds without integer overflow. |
| `saturating_sub(other)` | Subtracts without integer overflow. |
| `clamp(min, max)` | Bounds a value to an inclusive range. |

`Fixed` also implements `Add`, `Sub`, `Mul`, `Div`, `AddAssign`, `SubAssign`,
`Serialize`, and `Deserialize`.

## Simple Step API

`step(state, consumption_joules)` is the smallest deterministic state transition.

| Input | Type | Meaning |
| --- | --- | --- |
| `state` | `WorldState` | Current global simulation state. |
| `consumption_joules` | `Fixed` | Energy consumed during this tick. |

| Output | Meaning |
| --- | --- |
| `WorldState` | A new state with `tick` incremented and energy reduced to a zero floor. |

Behavior:

- increments `state.tick` by `1`
- subtracts `consumption_joules` from `state.energy_budget_joules`
- floors energy at `Fixed::ZERO`
- does not mutate the caller's original state because `WorldState` is passed by value

## World State

`WorldState` is the serializable global state used by both `step` and
`Simulation`.

| Field | Type | Description |
| --- | --- | --- |
| `tick` | `u64` | Current simulation tick. |
| `population` | `u64` | Aggregate population count. |
| `energy_budget_joules` | `Fixed` | Remaining energy budget. |
| `rng_seed` | `u64` | Seed for deterministic random behavior. |
| `factions` | `HashMap<u32, String>` | Faction ID to display name. |
| `faction_treasury` | `HashMap<u32, Fixed>` | Faction ID to treasury balance. |

`WorldState::default()` starts at tick `0`, population `1_000_000`, energy
budget `1_000_000_000_000`, RNG seed `42`, and three default factions.

## Simulation API

`Simulation` owns a `WorldState`, an ECS `hecs::World`, and a deterministic
`ChaCha8Rng`.

| API | Description |
| --- | --- |
| `Simulation::new()` | Creates a simulation with default seed `42` and starter entities. |
| `Simulation::with_seed(seed)` | Creates a simulation with a caller-supplied deterministic seed. |
| `tick()` | Advances production, citizen lifecycle, military, and economy phases. |
| `snapshot()` | Returns aggregate counts and energy state. |
| `rng_mut()` | Exposes the deterministic RNG for engine internals. |

Starter entities:

- 100 citizens
- 1 city center
- 5 farms
- 10 soldier units

`SimulationSnapshot` contains `tick`, `population`, `citizen_count`,
`building_count`, `military_count`, and `energy_budget`.

## Components

The engine exports ECS component types for the current simulation model.

| Component | Key fields |
| --- | --- |
| `Position` | `x`, `y` hex-grid coordinates. |
| `Citizen` | `age`, `health`, `ideology`, `welfare`, `job`. |
| `Building` | `building_type`, `hp`, `max_hp`, `position`. |
| `Resources` | `food`, `wood`, `metal`, `energy`. |
| `Production` | `output_type`, `rate`. |
| `MilitaryUnit` | `unit_type`, `strength`, `morale`, `position`, `faction_id`. |

Enums:

- `JobType`: `Farmer`, `Warrior`, `Scholar`, `Trader`, `Priest`, `Admin`, `Unemployed`
- `BuildingType`: `Farm`, `Mine`, `Barracks`, `Temple`, `Market`, `House`, `CityCenter`
- `ResourceType`: `Food`, `Wood`, `Metal`, `Energy`
- `UnitType`: `Soldier`, `Archer`, `Knight`, `Scout`

## Metrics

`metrics::compute(energy_budget_joules, consumption_joules)` returns:

| Field | Description |
| --- | --- |
| `waste_joules` | `consumption_joules * 0.1`, floored at zero. |
| `surplus_joules` | `energy_budget_joules - consumption_joules`, floored at zero. |
| `tyranny_index` | `consumption_joules / (energy_budget_joules + 1.0)`, capped at `1.0`. |
| `legitimacy_index` | `1.0 - tyranny_index`, floored at zero. |

## Policy

`effective_consumption(PolicyInput)` multiplies base consumption by a
non-negative scarcity multiplier.

| Field | Type | Description |
| --- | --- | --- |
| `base_consumption_joules` | `f64` | Baseline energy demand. |
| `scarcity_multiplier` | `f64` | Multiplier; negative values clamp to `0.0`. |

## I/O

The engine currently exposes minimal text persistence helpers:

| API | Description |
| --- | --- |
| `read_text(path)` | Reads UTF-8 text from a filesystem path. |
| `write_text(path, contents)` | Writes UTF-8 text to a filesystem path. |

Both return `std::io::Result`.

## Server Binary

`crates/server` is currently a smoke executable, not a long-running network
server. Running it performs one deterministic step and prints a metrics line:

```bash
cargo run -p civ-server
```

Example output shape:

```text
tick=1 energy=995000000000 waste=500000000 surplus=990000000000 tyranny=0.005025 legitimacy=0.994975
```

Use this binary as a quick wiring check for `civ-engine`, `step`, and
`metrics::compute`.

## Planned Protocol

The WebSocket JSON-RPC and binary-frame protocol is specified but not implemented
in the current codebase. Treat these as contracts for future work:

- `docs/specs/CIV-0200-client-protocol.md`
- `docs/models/civ-sim/API_EVENTS_SPEC.md`
- `docs/traceability/EVENT_TAXONOMY.md`

Do not depend on HTTP routes, WebSocket endpoints, `/healthz`, or `/metrics`
until the corresponding server implementation lands.
