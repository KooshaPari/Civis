# civ-economy

Conservation-complete economy layer for Civis (specs CIV-0100 / CIV-0107). Implements a double-entry ledger, allocation engines, district production, and conservation invariants.

## Overview

`civ-engine::Simulation::phase_economy` synchronises the joule energy budget into `EconomyState`, calls `drain_energy_budget` and `step`, then writes the updated state back to `WorldState`. All mutations are conservation-verified — resources are neither created nor destroyed, only transferred.

## Modules

| Module | Purpose |
|---|---|
| `allocation` | Priority-based resource allocation (planned, capitalist, labour-capacity regimes) |
| `allocator` | Generic allocator trait with `Bid` / `Offer` / `CancelledOrder` types |
| `currency_trust` | Currency acceptance and trust dynamics |
| `extraction` | Raw resource extraction from the world |
| `institution` | Institutional policies and governance |
| `market` | Market matching and price discovery |
| `stocks` | Stock/inventory tracking |
| `tax_policy` | Tax rate and revenue policy |
| `trade_flow` | Inter-district trade flow modelling |
| `trade_routes` | Route discovery and maintenance |

## Key Types

```rust
use civ_economy::{
    AllocationEngine, AllocationRegime, CapitalistAllocator,
    PlannedAllocator, EconomyState, CurrencyTrust,
};
```

## Usage

The economy crate is driven by the engine's tick loop. Direct usage:

```rust
use civ_economy::{step, drain_energy_budget, EconomyState};

let mut state = EconomyState::default();
drain_energy_budget(&mut state, joules);
step(&mut state, tick_number);
```

## Dependencies

- `serde` — Serialization
- `tracing` — Structured logging
- `proptest` (dev) — Property-based conservation invariant tests
