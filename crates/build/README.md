# build

> Procedural voxel building grammar with freehand authoring, producing a shared `BuildingGraph`.

## Overview

The `build` crate defines the procedural grammar and data structures for placing and constructing buildings in Civis 3D. It bridges two authoring modes: algorithmic placement driven by settlement simulation and freehand player placement, both converging on a single `BuildingGraph` representation.

Buildings progress through tiers, production chains, and construction sites, with an era-locked grammar that gates what can be built at each technological stage. Biome style bias ensures structures visually match their environment — timber frames in forests, stone in mountains, adobe in deserts.

Settlement clustering groups nearby parcels into logical districts, while emergent style keys capture the unique visual identity that arises from grammar rules interacting with local materials and era constraints.

## Features

- Procedural voxel building grammar with era-gated unlock tables
- Freehand player authoring with grammar validation
- Building tiers and production chain progression
- Construction site lifecycle management
- Biome-driven style bias for visual coherence
- Settlement district clustering
- Shared `BuildingGraph` output for rendering and simulation

## Usage

```rust
use build::*;
```

## Architecture

- **BuildingId** — Unique identifier for each placed building
- **Parcel** — Geographic region allocated to a building within a settlement
- **BuildingGraph** — Unified graph representation consumed by rendering and simulation
- **BuildingSpec** — Grammar-defined building archetype with tier, costs, and unlock conditions
- **BiomeStyleTag** — Biome-specific visual variant selector
- **EmergentStyleKey** — Composite key capturing the unique style that emerges from grammar + materials + era

The `BuildingGraph` is the central interchange format — both procedural and freehand authoring modes produce it, and it feeds into the rendering pipeline and economic simulation.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
