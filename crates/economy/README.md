# civ-economy

**Conservation-complete economy layer: double-entry ledger, allocation engines, and district production.**

The `civ-economy` crate implements the economic simulation for Civis, ensuring conservation invariants across the system. It handles resource allocation (joules, labor), currency trust, market dynamics, and trade routes, synchronizing with the main engine's energy budget.

## Key Types

- `EconomyState`: The central state of the economic system for a simulation tick.
- `AllocationEngine`: Trait and implementations for resource distribution strategies (e.g., `CapitalistAllocator`, `PlannedAllocator`).
- `JouleAllocator`: Specialized allocator for energy (joules).
- `CurrencyTrust`: Manages the acceptance and stability of currencies.

## Usage Example

The economy is typically stepped by the engine:

```rust
use civ_economy::{EconomyState, step};

let mut eco_state = EconomyState::default();
// ... initialize with data from WorldState ...

step(&mut eco_state);
// ... write back to WorldState ...
```

## Dependencies

This crate is designed to be lightweight:

- `serde`: Serialization.
- `tracing`: Logging and diagnostics.
