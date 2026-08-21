# needs

> Emergent life-sim needs model: per-agent needs decay, sickness, and death thresholds.

## Overview

The civ-needs crate implements the biological and psychological needs simulation for agents in Civis. Each agent carries a set of needs that decay over time; when thresholds are crossed the agent becomes sick or dies. All logic is deterministic and seeded for reproducible simulations.

## Features

- Per-agent needs model with configurable decay rates
- Sickness and death threshold mechanics
- Deterministic seeded RNG (ChaCha) for reproducibility
- Genetic modifiers via integration with civ-genetics
- Property-based tests for invariant checking
- Zero-allocation tick updates in the hot path

## Usage

```rust
use civ_needs::NeedsModel;

let mut model = NeedsModel::new(seed);
model.tick(agent, delta_time);
if model.is_sick(agent) {
    // apply sickness effects
}
```

## Architecture

Each agent owns a NeedsState with fields for hunger, thirst, sleep, social, and hygiene. The NeedsModel::tick method decays all needs by a per-agent rate modified by genetics. When a need crosses its critical threshold the agent transitions to a sick state. Sustained critical needs trigger the death pathway.

## License

Part of the Civis project.
