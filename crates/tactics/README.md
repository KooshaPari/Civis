# tactics

> Tactical voxel-destructible combat per-soldier + doctrine evolution genetic algorithm.

## Overview

The `tactics` crate provides a sophisticated combat simulation system for Civis. It handles individual soldier-level tactics, including A* and BFS pathfinding, formation management, and fog of war. It features a genetic algorithm for evolving military doctrines over time.

The system integrates deeply with the voxel world, supporting destructible terrain and line-of-sight calculations. It also bridges tactical outcomes to the broader game world through war economy drain and legend generation.

## Features

- Per-soldier tactical simulation with A*/BFS pathfinding
- Formation management and operational movement
- Fog of War and Line of Sight (LOS) calculations
- Genetic algorithm for Doctrine evolution
- War economy integration and bridge to Diplomacy/Legends

## Usage

```rust
use tactics::*;
```

## Architecture

`MilitaryUnit` represents individual soldiers, participating in `CombatEngagement` instances. `Doctrine` objects define the rules of engagement and are evolved via genetic algorithms. `WarBridge` and `FogOfWar` manage global state and visibility constraints.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
