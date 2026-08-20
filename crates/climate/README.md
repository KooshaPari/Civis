# climate

> Simplified climate model simulating CO2 concentration, mean temperature, and sea-level response to anthropogenic emissions.

## Overview

The `climate` crate provides a single-box energy-balance approach to modeling climate change within the Civis simulation. It tracks CO2 concentrations, global mean temperature anomalies, and sea-level rise in response to anthropogenic forcing, thermal inertia, and feedback loops.

Beyond the core thermodynamics, the crate includes modules for disaster spread (modeling the propagation of climate-related events), seasonal cycle dynamics, terrain erosion processes, and biome shifts driven by changing environmental conditions. This allows for a holistic representation of how climate change affects the simulated world's geography and ecology.

The model is designed to be deterministic and computationally efficient, making it suitable for long-running simulations where climate is one of many interacting systems.

## Features

- Single-box energy-balance climate model
- CO2 concentration and radiative forcing calculation
- Thermal inertia and feedback response
- Sea-level rise projection
- Disaster spread and propagation
- Seasonal cycle simulation
- Terrain erosion dynamics
- Biome shift modeling

## Usage

```rust
use climate::*;
```

## Architecture

- **ClimateState**: Holds the current state of the climate system (temperature, CO2, sea level).
- **ClimateParams**: Configuration parameters for the climate model (sensitivity, inertia).
- **DisasterGrid**: Manages the spatial propagation of climate disasters.
- **SeasonCycleParams**: Defines the parameters for seasonal variations.
- **ErosionGrid**: Tracks terrain erosion over time.
- **Biome**: Represents ecological zones and their transitions based on climate data.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.