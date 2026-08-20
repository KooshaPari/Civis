# civ-voxel

**Civis adapter over the shared `phenotype-voxel` kernel.**

The `civ-voxel` crate provides the 3D voxel world substrate for Civis, built on top of the shared `phenotype-voxel` engine. It implements adaptive voxel storage using Sparse Voxel Octrees (SVO) and dense 16³ leaf chunks, providing deterministic mesh generation and dirty-chunk tracking for efficient updates.

## Key Types

- `VoxelWorld`: The top-level structure representing the 3D voxel grid.
- `Chunk` / `ChunkCoord`: Represents a 16x16x16 block of voxels and its position.
- `VoxelOctree`: The sparse octree structure used for efficient spatial queries.
- `CubicMesher`: Implementation of the `Mesher` trait for generating geometry from voxels.

## Usage Example

```rust
use civ_voxel::{VoxelWorld, ChunkCoord, MaterialId};

let mut world = VoxelWorld::new();

// Place a block at a specific coordinate
let coord = ChunkCoord::new(0, 0, 0);
world.set_voxel(coord, MaterialId::new(1));

// Generate mesh for a chunk
// let mesh = world.mesh_chunk(coord);
```

## Dependencies

- [`phenotype-voxel`](https://github.com/KooshaPari/phenotype-gfx/tree/main/crates/phenotype-voxel): The shared voxel kernel from Phenotype-org.
- `bincode`: For efficient binary serialization of the dirty-chunk cache.
