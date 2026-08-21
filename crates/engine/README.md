# engine

> Core deterministic simulation engine using fixed-point arithmetic and ECS.

## Overview

The `engine` crate is the heart of the Civis simulation. It provides a full Entity Component System (ECS) based architecture, utilizing fixed-point arithmetic (`Fixed`) to ensure determinism across all platforms.

It manages the simulation loop, including policy and consumption, tyranny and legitimacy metrics, building emergence, climate, culture, disasters, diplomacy, economy, faction decisions, and god tools. It also handles the hash chain, integrity checks, replay/save functionality, scenario loading, spectator view, tech tree, and tutorial systems.

The engine is designed to be the central coordinator, pulling together all other crates to create a cohesive and deterministic simulation experience.

## Features

- Core ECS-based simulation loop
- Fixed-point arithmetic for determinism
- Full integration of climate, culture, diplomacy, economy, and more
- Building emergence and faction decisions
- God tools for player interaction
- Hash chain and integrity checks
- Replay and save/load functionality
- Scenario loading and spectator view
- Tech tree and tutorial integration

## Usage

```rust
use engine::*;
```

## Architecture

- **Simulation**: The main engine struct that manages the simulation loop and world state.
- **WorldState**: Represents the complete state of the game world at any given tick.
- **Fixed**: The fixed-point numeric type used for all calculations.
- **Building**: Represents structures within the simulation world.
- **Citizen**: The individual agents within the simulation.
- **MilitaryUnit**: Represents organized groups of citizens for defense/offense.
- **SimSeed**: Manages the deterministic seed for the simulation.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.