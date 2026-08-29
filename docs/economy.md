---
title: Economy
description: Joule budget, production, consumption, scarcity policy, and treasury model in Civis.
---

# Economy

## Overview

Civis uses a Joule-based energy economy: every productive activity spends Joules, and a global energy budget caps total production. On top of that, per-faction treasuries hold the surplus from citizen labor and tribute, and per-entity `Resources` track food, wood, metal, and energy stocks at the settlement level.

The economy subsystem lives in `crates/economy` and runs in the Economy tick phase, immediately after Physics and before AI. The economy phase is deterministic: given the same input state, it always produces the same output.

## Joule Budget

`WorldState.energy_budget_joules` is the global energy reservoir. Every productive action — building, training, construction, research — debits this budget. If the budget hits zero, productive actions queue but do not execute.

```rust
use civ_engine::{Fixed, WorldState, step};

let mut state = WorldState::default();
let next = step(state, Fixed::from_num(1_000));
```

`Fixed` arithmetic is used throughout so that the economy remains deterministic across platforms. The budget is floored at zero; it cannot go negative.

## Production

Production is computed per building per tick:

| Building | Cost (Joules) | Production | Maintenance |
|----------|---------------|------------|-------------|
| Farm | 1,000 | +10 food/tick | -1/tick |
| Mine | 2,000 | +5 metal/tick | -2/tick |
| Barracks | 3,000 | Unit training | -3/tick |
| Temple | 1,500 | +5 happiness | -1/tick |
| Market | 1,500 | Trade bonus | -1/tick |
| House | 500 | +10 pop cap | -0.5/tick |
| CityCenter | 10,000 | Governance | -5/tick |

The production rate is the lesser of available resources and energy budget. This is the bottleneck: a settlement that has food but no Joules cannot produce anything.

## Consumption

Citizens consume resources every tick based on their assigned `JobType`:

| Job | Food Production | Energy Cost | Special |
|-----|-----------------|-------------|---------|
| Farmer | +10 food/tick | -1/tick | — |
| Warrior | 0 | -2/tick | Combat |
| Scholar | 0 | -1/tick | Research |
| Trader | 0 | -1/tick | Trade routes |
| Priest | 0 | -1/tick | Happiness |
| Admin | 0 | -2/tick | Governance |
| Unemployed | -1 food | -0.5/tick | Unrest risk |

Citizen welfare (`Citizen.welfare`) drops when consumption exceeds production; chronic deficit risks citizen unrest and ideology drift.

## Scarcity Policy

The `policy::effective_consumption` function multiplies base consumption by a non-negative scarcity multiplier:

| Field | Type | Description |
|-------|------|-------------|
| `base_consumption_joules` | `f64` | Baseline energy demand. |
| `scarcity_multiplier` | `f64` | Multiplier; negative values clamp to `0.0`. |

The current policy is updated via the JSON-RPC method `sim.set_policy`:

```json
{ "scarcity_multiplier": 1.25, "base_consumption_joules": 1000.0 }
```

Validation rejects `NaN` and infinity; negative scarcity clamps to `0.0`.

## Treasury Model

Per-faction treasuries are stored in `WorldState.faction_treasury: HashMap<u32, Fixed>`. Treasury is updated by:

- Tribute treaty terms (`Tribute(resource, amount_per_tick)`).
- Trade route surplus, scaled by market and trade treaty bonuses.
- Building maintenance debits.
- War reparations when peace is signed.

A faction treasury at zero cannot propose new tribute treaties or hire new admins.

## Metrics

`metrics::compute(energy_budget_joules, consumption_joules)` returns:

| Field | Formula |
|-------|---------|
| `waste_joules` | `consumption_joules * 0.1`, floored at zero. |
| `surplus_joules` | `energy_budget_joules - consumption_joules`, floored at zero. |
| `tyranny_index` | `consumption_joules / (energy_budget_joules + 1.0)`, capped at `1.0`. |
| `legitimacy_index` | `1.0 - tyranny_index`, floored at zero. |

High tyranny reduces citizen welfare and increases the probability of unrest events.

## Trade Routes

Trade routes are persistent connections between two settlements. Each route generates a per-tick income based on the lower of the two endpoints' resource surplus, scaled by any active trade treaties. Routes can be disrupted during wartime, which halves the income for the duration of the conflict.

## See Also

- [Architecture](/architecture/) — economy crate and tick phase ordering.
- [Simulation](/simulation/) — `WorldState`, `step`, `Fixed` numeric type.
- [Diplomacy](/diplomacy/) — trade treaties, tribute terms, war reparations.
- [AI](/ai/) — citizen welfare and ideology reactions to economic state.
- [API](/api/) — `sim.set_policy` and treasury query methods.