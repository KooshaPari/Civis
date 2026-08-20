# civ-voxel

Adaptive voxel substrate for Civis. Re-exports the shared [`phenotype-voxel`](https://github.com/KooshaPari/phenotype-gfx/tree/main/crates/phenotype-voxel) kernel (sparse octree + dense 16-cubed leaf chunks) and adds Civis-side glue for ECS integration and protocol bindings.

## Overview

Part of the Civis 3D extension (`feat/civis-3d-foundation`). The core storage (SVO + dense leaf chunks), deterministic dirty queue, fixed-point coordinates, and per-engine `Mesher` trait live in the upstream `phenotype-voxel` crate. This crate re-exports the kernel verbatim and layers Civis-specific adapters on top.

## Re-exported Kernel Types

```rust
use civ_voxel::{
    VoxelOctree, VoxelWorld, Chunk, ChunkCoord, ChunkId, ChunkView,
    Mesher, CubicMesher, CubicVoxel, MeshBuffer, MeshResult, MeshVertex,
    MaterialId, MaterialPalette, VoxelMaterial,
    LodLevel, LodPolicy, select_lod,
    DirtyChunkEvent, OctreeNode, WriteSeq,
    WorldCoord, to_chunk_coord, VoxelScaleMultiplier, FIXED_SCALE,
};
```

## Modules

| Module | Purpose |
|---|---|
| `atlas` | Texture atlas management for meshing |

## Design Decisions

- **ADR-005** — Adaptive voxel with SVO for sparse regions and dense 16-cubed leaf chunks for populated areas
- **Deterministic dirty queue** — Ensures mesh rebuilds are reproducible across ticks
- **`#![forbid(unsafe_code)]`** — No unsafe in this crate

## Dependencies

| Crate | Role |
|---|---|
| `phenotype-voxel` | Shared voxel kernel (pinned git rev) |
| `bincode` | Deterministic binary serialisation for the dirty-chunk cache |
| `serde` | Serialization framework |

## Benchmarks

```bash
cargo bench -p civ-voxel        # ca_dirty_chunk + pbr_greedy_atlas
```
