# diffusion

> Bass/Rogers S-curve tech-adoption engine.

## Overview

The `diffusion` crate implements a pure deterministic math engine for tech adoption based on the Bass and Rogers S-curve models. It uses innovation coefficient `p` and imitation coefficient `q` to drive the propagation of technology across a population.

In the context of Civis, it drives the per-civilian wardrobe and tools era propagation, ensuring that technology spreads gradually rather than instantly. All math is performed in `f32` and is replay-deterministic with no RNG usage, ensuring bit-identical results across runs.

## Features

- Bass/Rogers S-curve adoption modeling
- Pure deterministic math (no RNG)
- Innovation and imitation coefficient tuning
- Gradual technology propagation
- Replay-deterministic state advancement

## Usage

```rust
use diffusion::*;
```

## Architecture

- **DiffusionParams**: Configuration for innovation (`p`) and imitation (`q`) coefficients.
- **tick_increase**: Calculates the increase in adoption for a given tick.
- **advance**: Advances the adoption state based on parameters.
- **trajectory**: Represents the full adoption curve over time.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.