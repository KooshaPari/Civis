# civ-engine

Deterministic simulation engine for the Civis godgame, using ECS architecture and fixed-point arithmetic (`i64` with scaling) for reproducible results across runs.

## Architecture

The engine orchestrates per-tick simulation phases — economy, diffusion, agent AI, building, and 3D voxel terrain updates — through a central `Simulation` struct backed by the `hecs` ECS. All arithmetic is deterministic: random generators are seeded via `rand_chacha`, and fractional values use fixed-point scaling.

### Key Types

| Type | Purpose |
|---|---|
| `Simulation` | Top-level simulation state; drives tick phases |
| `CivSaveBundle` | Serializable snapshot (compressed with `zstd`) for save/load |
| `Engine` | High-level facade over the simulation loop |

### Tick Phases

1. **Economy** — drains energy budget, runs allocation engines, writes back to `WorldState`
2. **Diffusion** — spreads resources across the voxel grid
3. **Needs / Agents** — evaluates agent desires, dispatches actions
4. **Build** — processes construction queues
5. **Voxel** — syncs the adaptive octree with world changes

## Usage

```rust
use civ_engine::{Simulation, CivSaveBundle};

let mut sim = Simulation::new(seed)?;
sim.tick()?;                // advance one tick
let bundle: CivSaveBundle = sim.save()?;  // serialize to disk
let restored = Simulation::load(&bundle)?;
```

## Dependencies

| Crate | Role |
|---|---|
| `civ-agents` | Agent AI and behavior trees |
| `civ-needs` | Maslow-style need evaluation |
| `civ-build` | Construction and building systems |
| `civ-diffusion` | Resource spread algorithms |
| `civ-voxel` | 3D voxel terrain (SVO + dense leaf chunks) |

## Key External Crates

- `hecs` — Entity Component System
- `rand` / `rand_chacha` — Seeded deterministic RNG
- `zstd` — Compression for save bundles
- `blake3` / `sha2` — Integrity hashing
