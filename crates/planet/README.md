# planet

> Deterministic single-planet geology, weather, day/night, and tides.

## Overview

The `planet` crate provides a pure, deterministic API for simulating planetary climate systems. It derives climate phases directly from a tick count, ensuring consistent results across sessions. This includes detailed geology maps, weather cell simulations, seasonal modifiers, and moon tidal offsets.

Designed for Civis, it offers Earth-like defaults while allowing deep customization of planetary parameters. The crate avoids side effects, making it ideal for parallel or multiplayer simulation where state synchronization is critical.

## Features

- Deterministic climate phase derivation from tick values
- Comprehensive geology mapping and weather cell simulation
- Seasonal modifiers and moon tidal offset calculations
- Earth-like defaults with customizable configurations
- Pure functional API suitable for parallel execution

## Usage

```rust
use planet::*;
```

## Architecture

The crate is structured around core configuration types (`PlanetConfig`, `MoonConfig`) that drive the simulation. The main entry points produce `Climate` states containing `GeologyMap` and `WeatherCell` data, modified by `SeasonalModifiers`. All computations are deterministic based on the input tick.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
