# civ-engine

**Deterministic simulation engine with ECS architecture and fixed-point arithmetic.**

The `civ-engine` crate provides the core simulation logic for the Civis project. It uses `hecs` for Entity Component System architecture and fixed-point arithmetic (i64 with scaling) to ensure deterministic simulation results across different environments.

## Key Types

- `Simulation`: The main engine struct that manages the simulation loop and world state.
- `WorldState`: Represents the complete state of the game world at any given tick.

## Usage Example

```rust
use civ_engine::Simulation;

// Create a new simulation with a specific seed for determinism
let mut sim = Simulation::new(42);

// Run the simulation for one tick
sim.tick();

// Access the world state
let world = sim.world();
```

## Dependencies

This crate relies on several sibling crates within the Civis workspace:

- [`civ-agents`](../agents): Agent behaviors and decision-making.
- [`civ-needs`](../needs): Needs and motivation system for agents.
- [`civ-build`](../build): Construction and building logic.
- [`civ-diffusion`](../diffusion): Chemical and energy diffusion.
- [`civ-voxel`](../voxel): 3D voxel world substrate.
