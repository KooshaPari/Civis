# voxel-bridge

> Public seam between voxel simulation and streaming/render clients.

## Overview

The `voxel-bridge` crate defines the public API contracts between the voxel simulation core and external streaming or rendering clients. It contains small, stable interface modules for various subsystems like boundary management, fluid CA, and LOD control.

As the internal implementations of these systems migrate from historical prototypes, this crate serves as the stable "seam" or facade. It ensures that client code remains decoupled from the rapidly evolving voxel engine internals.

## Features

- Stable contract modules for voxel subsystems
- Definitions for ChunkId, MaterialId, and WindowPolicy
- Abstract interfaces for Fluid CA and Worldgen
- PBR material and HUD overlays contracts
- Scale budget and streaming interfaces

## Usage

```rust
use voxel_bridge::*;
```

## Architecture

The crate is organized into modules representing different facets of the voxel system (e.g., `boundary`, `fluid`, `lod`). It exposes high-level types like `ChunkId` and `MaterialId` that are shared across the simulation and client boundaries.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
