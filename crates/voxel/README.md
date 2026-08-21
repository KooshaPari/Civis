# voxel

> Civis adapter over the shared phenotype-voxel kernel.

## Overview

The `voxel` crate is the primary interface for the Civis voxel world engine. It wraps the shared phenotype-voxel kernel (Sparse Voxel Octree + dense 16-cubed chunks) and adds Civis-specific features like a rich material palette, fluid cellular automata (CA), and procedural worldgen.

It manages LOD (Level of Detail), residency budgets, and PBR material blending. This crate is responsible for translating high-level world commands into efficient voxel data structures and operations.

## Features

- Sparse Voxel Octree (SVO) and dense chunk management
- 30+ material palette with PBR blending
- Fluid cellular automata (CA) simulation
- Procedural worldgen and brush systems
- LOD management and residency budgeting
- HUD overlay and boundary configuration

## Usage

```rust
use voxel::*;
```

## Architecture

The world is represented by `VoxelWorld`, composed of chunks identified by `ChunkId` and positioned in `WorldCoord`. Materials are referenced by `MaterialId`. Operations like `BrushStamp` allow for high-level modifications to the voxel data.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
